package main

import (
	"context"
	"flag"
	"log"
	pb "proyecto2/proto"
	"time"

	"github.com/gofiber/fiber/v2"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

type Student struct {
	Student    string `json:"student"`
	Age        int    `json:"age"`
	Faculty    string `json:"faculty"`
	Discipline int    `json:"discipline"`
}

var (
	athleticsAddr = flag.String("athleticsAddr", "athletics-service:50051", "athletics service address")
	swimmingAddr  = flag.String("athleticsAddr", "athletics-service:50051", "athletics service address")
	boxingAddr    = flag.String("athleticsAddr", "athletics-service:50051", "athletics service address")
)

func sendData(fiberCtx *fiber.Ctx) error {
	var body Student
	if err := fiberCtx.BodyParser(&body); err != nil {
		return fiberCtx.Status(400).JSON(fiber.Map{
			"error":   "Cannot parse body",
			"message": err.Error(),
		})
	}

	var serviceAddr string
	switch body.Discipline {
	case 1:
		serviceAddr = *athleticsAddr
	case 2:
		serviceAddr = *swimmingAddr
	case 3:
		serviceAddr = *boxingAddr
	default:
		return fiberCtx.Status(400).JSON(fiber.Map{
			"error": "Invalid discipline",
		})
	}

	conn, err := grpc.NewClient(serviceAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		return fiberCtx.Status(500).JSON(fiber.Map{
			"error":   "Cannot connect to service",
			"message": err.Error(),
		})
	}
	defer conn.Close()

	c := pb.NewStudentClient(conn)

	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()

	r, err := c.GetStudentReq(ctx, &pb.StudentRequest{
		Student:    body.Student,
		Faculty:    body.Faculty,
		Age:        int32(body.Age),
		Discipline: pb.Discipline(body.Discipline),
	})

	if err != nil {
		return fiberCtx.Status(500).JSON(fiber.Map{
			"error":   "Cannot get student",
			"message": err.Error(),
		})
	}

	return fiberCtx.Status(200).JSON(fiber.Map{
		"success": r.Success,
	})
}

func main() {
	app := fiber.New()
	app.Post("/agronomy", sendData)

	err := app.Listen(":8080")
	if err != nil {
		log.Println(err)
		return
	}
}
