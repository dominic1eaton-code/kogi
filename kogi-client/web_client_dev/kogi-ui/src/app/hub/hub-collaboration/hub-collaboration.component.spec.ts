import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubCollaborationComponent } from './hub-collaboration.component';

describe('HubCollaborationComponent', () => {
  let component: HubCollaborationComponent;
  let fixture: ComponentFixture<HubCollaborationComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubCollaborationComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubCollaborationComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
